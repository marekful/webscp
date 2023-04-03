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
    state.transfers.push(value);
    let stored = localStorage.getItem("rc-transfers");
    stored = stored ? JSON.parse(stored) : [];
    if (stored.indexOf(value.transferID) === -1) {
      stored.push(value.transferID);
      localStorage.setItem("rc-transfers", JSON.stringify(stored));
    }
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
    let stored = localStorage.getItem("rc-transfers");
    stored = stored ? JSON.parse(stored) : [];
    let idxToRemove = stored.indexOf(value);
    if (idxToRemove > -1) {
      stored.splice(idxToRemove, 1);
    }
    localStorage.setItem("rc-transfers", JSON.stringify(stored));
    localStorage.removeItem(`transfer-${value}`);
  },
  updateTransfer: (state, value) => {
    let transfers = [];
    let store, transferID;
    for (let idx = 0; idx < state.transfers.length; idx++) {
      if (state.transfers[idx].transferID === value.transferID) {
        [
          "agent",
          "pending",
          "items",
          "error",
          "status",
          "icon",
          "progress",
          "stats",
          "canceled",
          "cancelable",
          "showDetails",
        ].forEach(
          (attr) => {
            if (value[attr] !== undefined) {
              state.transfers[idx][attr] = value[attr];
            }
          }
        );
      }
      transfers.push(state.transfers[idx]);

      // update transfers in localStorage
      let {
        title, status, icon, action, agent, items, pending, canceled, error, stats,
      } = state.transfers[idx];
      store = { title, status, icon, action, agent, items, pending, canceled, error, stats };
      transferID = state.transfers[idx].transferID;
    }
    state.transfers = transfers;
    localStorage.setItem(`transfer-${transferID}`, JSON.stringify(store));
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
