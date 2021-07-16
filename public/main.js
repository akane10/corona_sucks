const BASE_URL = "https://covid.yapie.me";
const DROPDOWN_CONTENT = document.getElementById("drop-content");
const DROPDOWN = document.getElementById("drop");
const LOCATION = document.getElementById("location");
const TABLE_HEAD = document.getElementById("table-head");
const TABLE_BODY = document.getElementById("table-body");
const INPUT = document.getElementById("search");
const INFO = document.getElementById("info");
const LAST_UPDATED = document.getElementById("last-updated");

let TITLES = [];
let LISTS = [];
let DATA = {};
let CURRENT_INDEX = 0;
let DROPDOWN_SHOW = false;
let SELECTED_DATA = {};

function set_loading(bool) {
  if (bool) {
    document.getElementById("loading").style.display = "block";
  } else {
    document.getElementById("loading").style.display = "none";
  }
}

// https://stackoverflow.com/questions/3177836/how-to-format-time-since-xxx-e-g-4-minutes-ago-similar-to-stack-exchange-site
function timeSince(date) {
  const seconds = Math.floor((new Date() - date) / 1000);

  let interval = seconds / 31536000;

  if (interval > 1) {
    return Math.floor(interval) + " tahun";
  }
  interval = seconds / 2592000;
  if (interval > 1) {
    return Math.floor(interval) + " bulan";
  }
  interval = seconds / 86400;
  if (interval > 1) {
    return Math.floor(interval) + " hari";
  }
  interval = seconds / 3600;
  if (interval > 1) {
    return Math.floor(interval) + " jam";
  }
  interval = seconds / 60;
  if (interval > 1) {
    return Math.floor(interval) + " menit";
  }
  return Math.floor(seconds) + " detik";
}

async function get_last_updated() {
  try {
    LAST_UPDATED.innerHTML = `<p class="has-text-warning">data terakhir "sync" dari <a href="https://docs.google.com/spreadsheets/d/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c/edit#gid=0&fvid=2077488553" target="_blank">wargabantuwarga</a> . . .`;
    const get_data = LISTS.map(async (i) => {
      const res = await fetch(BASE_URL + "/data/" + i);
      const data = await res.json();
      return { title: data.title, updated_at: data.updated_at };
    });

    const data = await Promise.all(get_data);
    const updated = data
      .filter(Boolean)
      .reduce((acc, { title, updated_at }) => {
        const d1 = new Date(acc.updated_at || null);
        const d2 = new Date(updated_at || null);
        if (d1 > d2) {
          acc.title = acc.title;
          acc.updated_at = d1;
        } else {
          acc.title = title;
          acc.updated_at = d2;
        }

        return acc;
      }, {});

    LAST_UPDATED.innerHTML = `<p class="has-text-warning">data terakhir "sync" dari <a href="https://docs.google.com/spreadsheets/d/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c/edit#gid=0&fvid=2077488553" target="_blank">wargabantuwarga</a> ${timeSince(
      updated.updated_at
    )} yg lalu (${updated.title})</p>`;
  } catch (e) {
    console.log(e);
  }
}

function set_list() {
  fetch(BASE_URL + "/list")
    .then((response) => response.json())
    .then((data) => {
      LISTS = data;
      TITLES = data.map((title) => {
        return title.replace(/ /g, "").replace(".json", "");
      });
      TITLES.forEach((i, index) => {
        DROPDOWN_CONTENT.innerHTML += `<a class="dropdown-item" onclick="change_data(${index})">${i}</a>`;
      });
      const index_jkt = TITLES.findIndex((i) => i === "jkt");
      set_data(LISTS[index_jkt < 0 ? 0 : index_jkt]);
    })
    .catch((e) => {
      console.log(e);
      INFO.innerHTML = "";
      TABLE_HEAD.innerHTML = "";
      TABLE_BODY.innerHTML = "";
      LAST_UPDATED.innerHTML = "";

      INFO.innerHTML += `<h1 class="has-text-centered has-text-danger is-size-3">Ooppss Terjadi Error</h1>`;
    });
}
set_list();

function delay(callback, ms) {
  let timer = 0;
  return function () {
    let context = this,
      args = arguments;
    clearTimeout(timer);
    timer = setTimeout(function () {
      callback.apply(context, args);
    }, ms || 0);
  };
}

INPUT.addEventListener("keyup", delay(search, 1000));

function search() {
  const keyword = INPUT.value.toLowerCase();

  const data = DATA.data.filter((item) => {
    return item
      .map((ii) => ii.toLowerCase())
      .filter((ii) => ii.includes(keyword)).length;
  });

  SELECTED_DATA.data = data;
  render_data();
}

function show_dropdown() {
  if (DROPDOWN_SHOW) {
    DROPDOWN.classList.remove("is-active");
    DROPDOWN_SHOW = false;
  } else {
    DROPDOWN.classList.add("is-active");
    DROPDOWN_SHOW = true;
  }
}

function change_data(i) {
  set_data(LISTS[i]);
  show_dropdown();
}

function render_data() {
  TABLE_HEAD.innerHTML = "";
  TABLE_BODY.innerHTML = "";

  SELECTED_DATA.title.forEach((i) => {
    TABLE_HEAD.innerHTML += `<th>${i}</th>`;
  });
  SELECTED_DATA.data.forEach((i) => {
    let x = "<tr>";
    i.forEach((ii, index) => {
      if (ii.includes("http")) {
        x += `<td data-label="${SELECTED_DATA.title[index]}"><a href="${
          ii ? ii : "-"
        }" target="_blank">link</a></td>`;
      } else {
        x += `<td data-label="${SELECTED_DATA.title[index]}">${
          ii ? ii : "-"
        }</td>`;
      }
    });
    x += "</tr>";
    TABLE_BODY.innerHTML += x;
  });
}

function set_data(i) {
  set_loading(true);
  get_last_updated();
  fetch(BASE_URL + "/data/" + i)
    .then((response) => response.json())
    .then((data) => {
      SELECTED_DATA = {
        lokasi: data.title,
        title: data?.row_data[0] || [],
        data: data?.row_data.slice(1) || [],
      };
      DATA = { ...SELECTED_DATA };
      LOCATION.innerText = SELECTED_DATA.lokasi;

      set_loading(false);
      search();
      render_data();
    })
    .catch((e) => {
      console.log(e);
      set_loading(false);
      INFO.innerHTML = "";
      TABLE_HEAD.innerHTML = "";
      TABLE_BODY.innerHTML = "";
      LAST_UPDATED.innerHTML = "";

      INFO.innerHTML += `<h1 class="has-text-centered has-text-danger is-size-3">Ooppss Terjadi Error</h1>`;
    });
}
