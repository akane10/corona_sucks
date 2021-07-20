let CACHE_NAME = "pwgen-cache-v1";
let urlsToCache = [
  "./?v3",
  "Sick-Emoji.png",
  "index.html",
  "main.js",
  "style.css",
  "manifest.json",
  "sw.js",
  "https://cdnjs.cloudflare.com/ajax/libs/bulma/0.9.3/css/bulma.min.css",
];
console.log("loading sw");

self.addEventListener("install", function(event) {
  // Perform install steps
  console.log("installing sw");
  event.waitUntil(
    caches.open(CACHE_NAME).then(function(cache) {
      console.log("Opened cache");
      var x = cache.addAll(urlsToCache);
      console.log("cache added");
      return x;
    })
  );
});

self.addEventListener("fetch", function(event) {
  event.respondWith(
    caches.match(event.request).then(function(response) {
      // Cache hit - return response
      if (response) {
        return response;
      }
      return fetch(event.request);
    })
  );
});
