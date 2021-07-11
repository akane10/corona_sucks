const URL = "http://localhost:8000/data.json";
const DROPDOWN_CONTENT = document.getElementById("drop-content");
const DROPDOWN = document.getElementById("drop");
const TABLE_HEAD = document.getElementById("table-head");
const TABLE_BODY = document.getElementById("table-body");

let TITLES = [];
let DATA = [];
let DROPDOWN_SHOW = false;
let SELECTED_DATA = {};

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
  SELECTED_DATA = DATA[i];
  render_data();
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
    i.forEach((ii) => {
      if (ii.includes("http")) {
        x += `<td><a href="${ii}">link</a></td>`;
      } else {
        x += `<td>${ii}</td>`;
      }
    });
    x += "</tr>";
    TABLE_BODY.innerHTML += x;
  });
}

fetch(URL)
  .then((response) => response.json())
  .then((data) => {
    TITLES = data.map(({ title }) => title);
    DATA = data.map((i) => {
      return {
        lokasi: i.title,
        title: i?.row_data[0],
        data: i?.row_data.slice(1),
      };
    });
    TITLES.forEach((i, index) => {
      DROPDOWN_CONTENT.innerHTML += `<a class="dropdown-item" onclick="change_data(${index})">${i}</a>`;
    });

    change_data(0);
    render_data();
    show_dropdown();
  });