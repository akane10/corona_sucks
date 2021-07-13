const BASE_URL = "https://covid.yapie.me";
const DROPDOWN_CONTENT = document.getElementById("drop-content");
const DROPDOWN = document.getElementById("drop");
const LOCATION = document.getElementById("location");
const TABLE_HEAD = document.getElementById("table-head");
const TABLE_BODY = document.getElementById("table-body");
const INPUT = document.getElementById("search");

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
    });
}
