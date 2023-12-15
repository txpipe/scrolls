import("scrolls:reducer").then(({ reduce }) => {
  globalThis["reduce"] = reduce;
});
