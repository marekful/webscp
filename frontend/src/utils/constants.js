const name = window.WebSCP.Name || "WebSCP";
const disableExternal = window.WebSCP.DisableExternal;
const disableUsedPercentage = window.WebSCP.DisableUsedPercentage;
const baseURL = window.WebSCP.BaseURL;
const staticURL = window.WebSCP.StaticURL;
const recaptcha = window.WebSCP.ReCaptcha;
const recaptchaKey = window.WebSCP.ReCaptchaKey;
const signup = window.WebSCP.Signup;
const version = window.WebSCP.Version;
const logoURL = `${staticURL}/img/logo.svg`;
const noAuth = window.WebSCP.NoAuth;
const authMethod = window.WebSCP.AuthMethod;
const loginPage = window.WebSCP.LoginPage;
const theme = window.WebSCP.Theme;
const enableThumbs = window.WebSCP.EnableThumbs;
const resizePreview = window.WebSCP.ResizePreview;
const enableExec = window.WebSCP.EnableExec;
const origin = window.location.origin;

export {
  name,
  disableExternal,
  disableUsedPercentage,
  baseURL,
  logoURL,
  recaptcha,
  recaptchaKey,
  signup,
  version,
  noAuth,
  authMethod,
  loginPage,
  theme,
  enableThumbs,
  resizePreview,
  enableExec,
  origin,
};
