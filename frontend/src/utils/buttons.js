function loading(button) {
  let el = document.querySelector(`#${button}-button > i`);

  if (el === undefined || el === null) {
    console.log('Error getting button ' + button) // eslint-disable-line
    return;
  }

  if (el.innerHTML == "autorenew" || el.innerHTML == "done") {
    return;
  }

  el.dataset.icon = el.innerHTML;
  el.style.opacity = 0;

  setTimeout(() => {
    el.classList.add("spin");
    el.innerHTML = "autorenew";
    el.style.opacity = 1;
  }, 100);
}

function loadingPromise(button) {
  return new Promise(function (resolve) {
    let el = document.querySelector(`#${button}-button > i`);

    if (el === undefined || el === null) {
      console.log('Error getting button ' + button) // eslint-disable-line
      return;
    }

    if (el.innerHTML == "autorenew" || el.innerHTML == "done") {
      return;
    }

    el.dataset.icon = el.innerHTML;
    el.style.opacity = 0;

    setTimeout(() => {
      el.classList.add("spin");
      el.innerHTML = "autorenew";
      el.style.opacity = 1;

      resolve();
    }, 100);
  });
}

function done(button) {
  let el = document.querySelector(`#${button}-button > i`);

  if (el === undefined || el === null) {
    console.log('Error getting button ' + button) // eslint-disable-line
    return;
  }

  el.style.opacity = 0;

  setTimeout(() => {
    el.classList.remove("spin");
    el.innerHTML = el.dataset.icon;
    el.style.opacity = 1;
  }, 100);
}

function donePromise(button) {
  return new Promise(function (resolve) {
    let el = document.querySelector(`#${button}-button > i`);

    if (el === undefined || el === null) {
      console.log('Error getting button ' + button) // eslint-disable-line
      return;
    }

    el.style.opacity = 0;

    setTimeout(() => {
      el.classList.remove("spin");
      el.innerHTML = el.dataset.icon;
      el.style.opacity = 1;

      resolve();
    }, 100);
  });
}

function success(button) {
  let el = document.querySelector(`#${button}-button > i`);

  if (el === undefined || el === null) {
    console.log('Error getting button ' + button) // eslint-disable-line
    return;
  }

  el.style.opacity = 0;

  setTimeout(() => {
    el.classList.remove("spin");
    el.innerHTML = "done";
    el.style.opacity = 1;

    setTimeout(() => {
      el.style.opacity = 0;

      setTimeout(() => {
        el.innerHTML = el.dataset.icon;
        el.style.opacity = 1;
      }, 100);
    }, 500);
  }, 100);
}

function successPromise(button) {
  return new Promise(function (resolve, reject) {
    let el = document.querySelector(`#${button}-button > i`);

    if (el === undefined || el === null) {
      console.log("Error getting button " + button) // eslint-disable-line
      return reject("Error getting button " + button);
    }

    el.style.opacity = 0;

    setTimeout(() => {
      el.classList.remove("spin");
      el.innerHTML = "done";
      el.style.opacity = 1;

      setTimeout(() => {
        el.style.opacity = 0;

        setTimeout(() => {
          el.innerHTML = el.dataset.icon;
          el.style.opacity = 1;
          resolve();
        }, 100);
      }, 500);
    }, 100);
  });
}

function active(button, active = true) {
  let el = document.querySelector(`#${button}-button`);

  if (el === undefined || el === null) {
    console.log('Error getting button ' + button) // eslint-disable-line
    return;
  }

  el.classList[active ? "add" : "remove"]("active");
}

function icon(button, icon) {
  let el = document.querySelector(`#${button}-button > i`);

  if (el === undefined || el === null) {
    console.log('Error getting button ' + button) // eslint-disable-line
    return;
  }

  el.innerHTML = icon;
}

export default {
  loading,
  loadingPromise,
  done,
  donePromise,
  success,
  successPromise,
  active,
  icon,
};
