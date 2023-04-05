import buttons from "./buttons";
import i18n from "@/i18n";

function create(
  $store,
  transferID,
  transfers,
  action,
  agent,
  items,
  status = "Starting",
  icon = "folder_zip",
  stats = { progress: [], total: [] },
  pending = true,
  canceled = false,
  error = false,
  uploading = false
) {
  status = status || "starting";
  stats = stats || { progress: [], total: [] };
  let cancelable = true;
  let showDetails = false;
  let sseClient;
  let title = "";
  switch (action) {
    case "copy":
      title = i18n.t("transfer.copying");
      break;
    case "move":
      title = i18n.t("transfer.moving");
      break;
  }
  let plural = items.length > 1 ? "s" : "";
  title += ` ${items.length} item${plural} to ${agent.host}:${agent.port}`;

  if (!canceled && pending) {
    sseClient = new EventSource(`/api/sse/transfers/${transferID}/poll`);
    sseClient.onmessage = handleMessage($store);
    sseClient.onerror = handleError($store);
    sseClient.transferID = transferID;
  }

  let data = {
    agent,
    action,
    transferID,
    sseClient,
    title,
    pending,
    canceled,
    error,
    status,
    icon,
    items,
    stats,
    uploading,
    cancelable,
    showDetails,
  };
  $store.commit("addTransfer", data);
  storeAdd(data);
}

function get(transfers, transferID) {
  for (let transfer of transfers) {
    if (transfer.transferID === transferID) {
      return transfer;
    }
  }
  return null;
}

function remove($store, transferID) {
  $store.commit("removeTransfer", transferID);
  storeRemove(transferID);
}

function update($store, data) {
  let newTransfers = [];
  let store;

  for (let transfer of $store.state.transfers) {
    let newTransfer = Object.fromEntries(Object.entries(transfer));
    if (transfer.transferID === data.transferID) {
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
        "uploading",
        "showDetails",
      ].forEach((attr) => {
        if (data[attr] !== undefined) {
          newTransfer[attr] = data[attr];
        }
      });

      let {
        transferID,
        title,
        status,
        icon,
        action,
        agent,
        items,
        pending,
        canceled,
        uploading,
        error,
        stats,
      } = newTransfer;
      store = {
        transferID,
        title,
        status,
        icon,
        action,
        agent,
        items,
        pending,
        canceled,
        uploading,
        error,
        stats,
      };
      storeUpdate(store);
    }
    newTransfers.push(newTransfer);
  }

  $store.commit("replaceTransfers", newTransfers);
}

function storeAdd(data) {
  let stored = localStorage.getItem("rc-transfers");
  stored = stored ? JSON.parse(stored) : [];
  if (stored.indexOf(data.transferID) === -1) {
    stored.push(data.transferID);
    localStorage.setItem("rc-transfers", JSON.stringify(stored));
  }
  storeUpdate(data);
}

function storeRemove(transferID) {
  let stored = localStorage.getItem("rc-transfers");
  stored = stored ? JSON.parse(stored) : [];
  let idxToRemove = stored.indexOf(transferID);
  if (idxToRemove > -1) {
    stored.splice(idxToRemove, 1);
  }
  localStorage.setItem("rc-transfers", JSON.stringify(stored));
  localStorage.removeItem(`transfer-${transferID}`);
}

function storeUpdate(data) {
  if (!data.transferID) return;
  localStorage.setItem(`transfer-${data.transferID}`, JSON.stringify(data));
}

function setButtonActive(transfers) {
  let pending = 0;
  let error = 0;
  for (let tr of transfers) {
    if (tr.pending) {
      pending += 1;
    }
    if (tr.error) {
      error += 1;
    }
  }

  buttons.active("transfers", pending > 0);

  buttons[pending > 0 ? "loadingPromise" : "donePromise"]("transfers").then(
    () => {
      buttons.icon(
        "transfers",
        error > 0 && pending === 0 ? "sync_problem" : "sync"
      );
    }
  );
}

function handleError($store) {
  return function (event) {
    console.log("SSE Error > ", event, $store.state.transfers);
  };
}

function handleMessage($store) {
  return function (event) {
    if (!event.isTrusted) return;

    let icon,
      data,
      message,
      stats,
      messageTr,
      extra = "",
      errorMessage,
      pending = true,
      canceled = false,
      cancelable = true,
      uploading = false;

    if (event.data.indexOf("::") !== -1) {
      let s = event.data.split("::");
      message = s[0];
      data = s[1];
      if (typeof s[2] === "string") {
        extra = s[2];
      }
    } else {
      message = event.data;
    }
    messageTr = message;
    switch (message) {
      case "archiving":
        icon = "folder_zip";
        cancelable = false;
        break;
      case "starting upload":
        icon = "drive_folder_upload";
        messageTr = "startingUpload";
        uploading = true;
        break;
      case "uploading":
        icon = "drive_folder_upload";
        uploading = true;
        break;
      case "extracting":
        icon = "drive_file_move";
        cancelable = false;
        break;
      case "complete":
        icon = "done";
        pending = false;
        break;
      case "progress":
        icon = "drive_folder_upload";
        uploading = true;
        messageTr = "uploading";
        if (data === "stats") {
          message = "uploading";
          stats = getStats(extra);
        }
        break;
      case "signal":
        messageTr = extra;
        pending = false;
        canceled = true;
        uploading = false;
        icon = "highlight_off";
        break;
      default:
        // error case
        icon = "error_outline";
        uploading = false;
        errorMessage = i18n.te(`transfer.${messageTr}`)
          ? i18n.t(`transfer.${messageTr}`)
          : message;

        update($store, {
          transferID: event.target.transferID,
          pending: false,
          error: true,
          status: errorMessage,
          icon,
          cancelable,
          uploading,
        });

        setButtonActive($store.state.transfers);

        return;
    }

    update($store, {
      transferID: event.target.transferID,
      status: i18n.t(`transfer.${messageTr}`),
      pending,
      icon,
      stats,
      canceled,
      cancelable,
      uploading,
    });

    if (pending) {
      return;
    }
    buttons
      .successPromise("transfers")
      .finally(() => setButtonActive($store.state.transfers));
  };
}

function getStats(data) {
  let bytes = data.split("/");
  let progress = bytes[0];
  let total = bytes[1];
  let result = {};

  if (progress < 1024 * 1024) {
    result.progress = [...(progress / 1024).toFixed(2).split("."), "KB"];
  } else if (progress < 1024 * 1024 * 1024) {
    result.progress = [...(progress / 1024 / 1024).toFixed(2).split("."), "MB"];
  } else {
    result.progress = [
      ...(progress / 1024 / 1024 / 1024).toFixed(2).split("."),
      "GB",
    ];
  }
  if (total < 1024 * 1024) {
    result.total = [...(total / 1024).toFixed(2).split("."), "KB"];
  } else if (total < 1024 * 1024 * 1024) {
    result.total = [...(total / 1024 / 1024).toFixed(2).split("."), "MB"];
  } else {
    result.total = [
      ...(total / 1024 / 1024 / 1024).toFixed(2).split("."),
      "GB",
    ];
  }

  return result;
}

export default {
  create,
  get,
  update,
  remove,
  setButtonActive,
};
