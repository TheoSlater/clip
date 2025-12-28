const { heroui } = require("@heroui/react");
const { darkColors, lightColors } = require("./theme");

/** @type {import('tailwindcss').Config} */
const config = {
    content: [
        "./src/**/*.{js,ts,jsx,tsx}",
        "./node_modules/@heroui/theme/dist/**/*.{js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {},
    },
    darkMode: "class",
    plugins: [
        heroui({
            themes: {
                dark: {
                    colors: darkColors,
                },
            },
        }),
    ],
};

module.exports = config;
