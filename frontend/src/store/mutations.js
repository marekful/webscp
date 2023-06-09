import * as i18n from "@/i18n";
import moment from "moment";

const mutations = {
  closeHovers: (state) => {
    state.show = null;
    state.showConfirm = null;
  },
  toggleShell: (state) => {
    state.showShell = !state.showShell;
  },
  toggleDotfiles: (state) => {
    state.showDotfiles = !state.showDotfiles;
  },
  showHover: (state, value) => {
    if (typeof value !== "object") {
      state.show = value;
      return;
    }

    state.show = value.prompt;
    state.showConfirm = value.confirm;
  },
  showError: (state) => {
    state.show = "error";
  },
  showSuccess: (state) => {
    state.show = "success";
  },
  setLoading: (state, value) => {
    state.loading = value;
  },
  setReload: (state, value) => {
    state.reload = value;
  },
  setUser: (state, value) => {
    if (value === null) {
      state.user = null;
      return;
    }

    let locale = value.locale;

    if (locale === "") {
      locale = i18n.detectLocale();
    }

    moment.locale(locale);
    i18n.default.locale = locale;
    state.user = value;
  },
  setLoginAgent: (state, value) => {
    state.loginAgent = value;
  },
  resetLoginAgent: (state) => {
    state.loginAgent = null;
  },
  setJWT: (state, value) => (state.jwt = value),
  multiple: (state, value) => (state.multiple = value),
  addSelected: (state, value) => state.selected.push(value),
  removeSelected: (state, value) => {
    let i = state.selected.indexOf(value);
    if (i === -1) return;
    state.selected.splice(i, 1);
  },
  resetSelected: (state) => {
    state.selected = [];
  },
  addTransfer: (state, value) => {
    state.transfers.unshift(value);
  },
  replaceTransfers: (state, value) => {
    state.transfers = value;
  },
  removeTransfer: (state, value) => {
    for (let idx = 0; idx < state.transfers.length; idx++) {
      if (state.transfers[idx].transferID === value) {
        if (state.transfers[idx].sseClient) {
          state.transfers[idx].sseClient.close();
          delete state.transfers[idx].sseClient;
        }
        state.transfers.splice(idx, 1);
        break;
      }
    }
  },
  updateUser: (state, value) => {
    if (typeof value !== "object") return;

    for (let field in value) {
      if (field === "locale") {
        moment.locale(value[field]);
        i18n.default.locale = value[field];
      }

      state.user[field] = value[field];
    }
  },
  updateRequest: (state, value) => {
    state.oldReq = state.req;
    state.req = value;
  },
  updateClipboard: (state, value) => {
    state.clipboard.key = value.key;
    state.clipboard.items = value.items;
    state.clipboard.path = value.path;
  },
  resetClipboard: (state) => {
    state.clipboard.key = "";
    state.clipboard.items = [];
  },
};

export default mutations;
