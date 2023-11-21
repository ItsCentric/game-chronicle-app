module.exports = {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx,svelte}"],
  theme: {
    extend: {},
  },
  daisyui: {
    themes: ["night"],
  },
  plugins: [require("daisyui")],
};
