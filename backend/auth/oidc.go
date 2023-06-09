package auth

import (
	"context"
	"crypto/rand"
	"fmt"
	"log"
	"math"
	"math/big"
	"net/http"
	"net/url"
	"os"
	"strings"

	"github.com/coreos/go-oidc/v3/oidc"
	"golang.org/x/oauth2"

	"github.com/marekful/webscp/settings"
	"github.com/marekful/webscp/users"
)

// MethodOIDCAuth is used to identify oidc auth.
const MethodOIDCAuth settings.AuthMethod = "oidc"

// OIDCAuth is an Open ID Connect auther implementation.
type OIDCAuth struct {
	OIDC *OAuthClient `json:"oidc" yaml:"oidc"`
}

// Auth is executed when the identity provider enters the callback phase of an oauth code flow.
func (a OIDCAuth) Auth(r *http.Request, usr users.Store, _ *settings.Settings, srv *settings.Server) (*users.User, error) {
	cookie, _ := r.Cookie("auth")
	if cookie != nil {
		return nil, nil
	}

	log.Println("oidc auth callback")
	u, err := a.OIDC.HandleAuthCallback(r, usr, srv)

	return u, err
}

// LoginPage tells that oidc auth doesn't require a login page.
func (a OIDCAuth) LoginPage() bool {
	return false
}

// ConfigChanged tells if the provided config values are different compared to saved values.
func (a OIDCAuth) ConfigChanged(config map[string]string) bool {
	if a.OIDC == nil {
		return true
	}

	return config["clientid"] != a.OIDC.ClientID ||
		config["clientsecret"] != a.OIDC.ClientSecret ||
		config["issuer"] != a.OIDC.Issuer ||
		config["redirecturl"] != a.OIDC.RedirectURL
}

// OAuthClient describes the oidc connector parameters.
type OAuthClient struct {
	ClientID               string                `json:"clientID"`
	ClientSecret           string                `json:"clientSecret"`
	Issuer                 string                `json:"issuer"`
	RedirectURL            string                `json:"redirectURL"`
	RedirectURLAppendQuery bool                  `json:"redirectURLAppendQuery"`
	OAuth2Config           oauth2.Config         `json:"-"`
	Verifier               *oidc.IDTokenVerifier `json:"-"`
}

// InitClient configures the connector via oidc discovery.
func (o *OAuthClient) InitClient() {
	ctx := context.Background()
	provider, err := oidc.NewProvider(ctx, o.Issuer)
	if err != nil {
		log.Fatal(err)
	}

	o.Verifier = provider.Verifier(&oidc.Config{ClientID: o.ClientID})
	o.OAuth2Config = oauth2.Config{
		ClientID:     o.ClientID,
		ClientSecret: o.ClientSecret,
		RedirectURL:  o.RedirectURL,
		Endpoint:     provider.Endpoint(),
		Scopes:       []string{oidc.ScopeOpenID, "profile", "email"},
	}
}

// InitAuthFlow triggers the oidc authentication flow.
func (o *OAuthClient) InitAuthFlow(w http.ResponseWriter, r *http.Request) {
	o.InitClient()

	rand1, _ := rand.Int(rand.Reader, big.NewInt(math.MaxInt32))
	rand2, _ := rand.Int(rand.Reader, big.NewInt(math.MaxInt32))
	state := fmt.Sprintf("%x", rand1)
	nonce := fmt.Sprintf("%x", rand2)
	if strings.HasPrefix(r.URL.Path, "/files/") && o.RedirectURLAppendQuery {
		o.OAuth2Config.RedirectURL += "?redirect=" + url.QueryEscape(r.URL.Path)
	}
	redirect := o.OAuth2Config.AuthCodeURL(state, oidc.Nonce(nonce))

	log.Println("oidc init flow ", redirect)
	w.Header().Set("Set-Cookie", "state="+state+"; path=/")
	http.Redirect(w, r, redirect, http.StatusMovedPermanently)
}

// HandleAuthCallback manages code exchange and obtains the id token.
func (o *OAuthClient) HandleAuthCallback(r *http.Request, usr users.Store, srv *settings.Server) (*users.User, error) {
	o.InitClient()
	code := r.URL.Query().Get("code")
	stateQuery := r.URL.Query().Get("state")
	stateCookie, err := r.Cookie("state")

	// Validate state
	if code == "" || stateQuery == "" || err != nil || stateQuery != stateCookie.Value {
		log.Println("oidc invalid callback request")
		return nil, os.ErrPermission
	}

	// Exchange code for token
	oauth2Token, err := o.OAuth2Config.Exchange(context.Background(), code)
	if err != nil {
		log.Printf("oidc code exchange failed: %s", err)
		return nil, os.ErrPermission
	}

	// Parse id token
	rawIDToken, ok := oauth2Token.Extra("id_token").(string)
	if !ok {
		log.Println("oidc id token extract failed")
		return nil, os.ErrPermission
	}

	// Verify id token
	idToken, err := o.Verifier.Verify(context.Background(), rawIDToken)
	if err != nil {
		log.Printf("oidc token verify failed: %s", err)
		return nil, os.ErrPermission
	}

	// Extract claims
	var claims struct {
		Username string `json:"preferred_username"`
	}
	if errClaims := idToken.Claims(&claims); errClaims != nil {
		log.Printf("oidc extract claims failed: %s", errClaims)
		return nil, os.ErrPermission
	}

	// Find filebrowser user by oidc username
	u, err := usr.Get(srv.Root, claims.Username)
	if err != nil {
		log.Println("oidc authenticated but no matching filebrowser user")
		return nil, os.ErrPermission
	}
	u.AuthSource = "oidc"
	log.Printf("oidc authenticated user %s", u.Username)

	return u, nil
}
