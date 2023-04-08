const BASE_URL = "https://covid.yapie.me";
const DROPDOWN_CONTENT = document.getElementById("drop-content");
const DROPDOWN = document.getElementById("drop");
const LOCATION = document.getElementById("location");
const TABLE_HEAD = document.getElementById("table-head");
const TABLE_BODY = document.getElementById("table-body");
const INPUT = document.getElementById("search");
const INFO = document.getElementById("info");
const LAST_UPDATED = document.getElementById("last-updated");
const NEXT_PAGE = document.getElementById("next-page");
const PREVIOUS_PAGE = document.getElementById("previous-page");
const FIRST_PAGE = document.getElementById("first-page");
const LAST_PAGE = document.getElementById("last-page");
const CURRENT_PAGE_DOC = document.getElementById("current-page");

let TITLES = [];
let DATA = {};
let CURRENT_INDEX = 0;
let DROPDOWN_SHOW = false;
let SELECTED_DATA = {};
let CURRENT_PAGE = 1;
let TOTAL_PAGES = 1;

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
    LAST_UPDATED.innerHTML = `<p class="has-text-warning">data terakhir "sync" dari <a href="https://docs.google.com/spreadsheets/d/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c/edit#gid=0&fvid=2077488553" rel="noreferrer" target="_blank">wargabantuwarga</a> . . .`;
    const res = await fetch(BASE_URL + "/data/last_updated.json");
    const data = await res.json();

    LAST_UPDATED.innerHTML = `<p class="has-text-warning">data terakhir "sync" dari <a href="https://docs.google.com/spreadsheets/d/1RIcSiQqPCw-6H55QIYwblIQDPpFQmDNC73ukFa05J7c/edit#gid=0&fvid=2077488553" rel="noreferrer" target="_blank">wargabantuwarga</a> ${timeSince(
      new Date(data.updated_at)
    )} yg lalu (${data.title.trim()})</p>`;
  } catch (e) {
    console.log(e);
  }
}

function set_list() {
  fetch(BASE_URL + "/list")
    .then((response) => response.json())
    .then((data) => {
      data.forEach(({ title, sheet_id }) => {
        DROPDOWN_CONTENT.innerHTML += `<a class="dropdown-item" onclick="change_data(${sheet_id})">${title}</a>`;
      });
      set_data("0");
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
NEXT_PAGE.addEventListener("click", (el) => {
  if (el.target.attributes.disabled) return
  CURRENT_PAGE += 1;
  render_data();
});
PREVIOUS_PAGE.addEventListener("click", (el) => {
  if (el.target.attributes.disabled) return
  CURRENT_PAGE -= 1;
  render_data();
});
FIRST_PAGE.addEventListener("click", (el) => {
  if (el.target.attributes.disabled) return
  CURRENT_PAGE = 1;
  render_data();
});
LAST_PAGE.addEventListener("click", (el) => {
  if (el.target.attributes.disabled) return
  CURRENT_PAGE = TOTAL_PAGES;
  render_data();
});

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
  set_data(i);
  show_dropdown();
}

function set_pagination_ui() {
  if (CURRENT_PAGE >= SELECTED_DATA.pages.total) {
    NEXT_PAGE.setAttribute("disabled", "");
    LAST_PAGE.setAttribute("disabled", "");
  } else {
    NEXT_PAGE.removeAttribute("disabled");
    LAST_PAGE.removeAttribute("disabled");
  }
  if (CURRENT_PAGE <= 1) {
    PREVIOUS_PAGE.setAttribute("disabled", "");
    FIRST_PAGE.setAttribute("disabled", "");
  } else {
    PREVIOUS_PAGE.removeAttribute("disabled");
    FIRST_PAGE.removeAttribute("disabled");
  }
  CURRENT_PAGE_DOC.innerHTML = CURRENT_PAGE;
  LAST_PAGE.innerHTML = TOTAL_PAGES;
}

function render_data() {
  TABLE_HEAD.innerHTML = "";
  TABLE_BODY.innerHTML = "";
  INFO.innerHTML = "";
  set_pagination_ui();

  SELECTED_DATA.title.forEach((i) => {
    TABLE_HEAD.innerHTML += `<th>${i}</th>`;
  });
  (SELECTED_DATA.pages.data[CURRENT_PAGE] || []).forEach((i) => {
    let x = "<tr>";
    i.forEach((ii, index) => {
      if (ii.includes("http")) {
        x += `<td data-label="${SELECTED_DATA.title[index]}"><a href="${
          ii ? ii : "-"
        }" rel="noreferrer" target="_blank">link</a></td>`;
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

function set_pages(data) {
  const total = data.length ? Math.ceil(data.length / 10) : 0;
  let start = 0;
  let end = 10;
  let obj = {};

  for (let i = 1; i <= total; i++) {
    obj[i] = data.slice(start, end);
    start += 9;
    end += 10;
  }

  return { total, data: obj };
}

function set_data(i) {
  set_loading(true);
  get_last_updated();
  fetch(BASE_URL + "/data/" + i + ".json")
    .then((response) => response.json())
    .then((data) => {
      CURRENT_PAGE = 1
      const title = data?.row_data[0] || [];
      const data_ = data?.row_data.slice(1) || [];
      SELECTED_DATA = {
        lokasi: data.title,
        title,
        data: data_,
        pages: set_pages(data_),
      };
      DATA = { ...SELECTED_DATA };
      LOCATION.innerText = SELECTED_DATA.lokasi;

      // console.log({ SELECTED_DATA });

      TOTAL_PAGES = SELECTED_DATA.pages.total;
      search();
      set_loading(false);
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
