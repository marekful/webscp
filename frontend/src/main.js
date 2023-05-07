import "whatwg-fetch";
import cssVars from "css-vars-ponyfill";
import { sync } from "vuex-router-sync";
import store from "@/store";
import router from "@/router";
import i18n from "@/i18n";
import Vue from "@/utils/vue";
import { recaptcha, loginPage, authMethod } from "@/utils/constants";
import { login, validateLogin } from "@/utils/auth";
import App from "@/App";

cssVars();

sync(store, router);

// start application
async function start() {
  try {
    if (loginPage || authMethod === "oidc") {
      await validateLogin();
    } else {
      await login("", "", "");
    }
  } catch (e) {
    console.log(e);
  }

  if (recaptcha) {
    await new Promise((resolve) => {
      const check = () => {
        if (typeof window.grecaptcha === "undefined") {
          setTimeout(check, 100);
        } else {
          resolve();
        }
      };

      check();
    });
  }

  new Vue({
    el: "#app",
    store,
    router,
    i18n,
    template: "<App/>",
    components: { App },
  });
}

start();
