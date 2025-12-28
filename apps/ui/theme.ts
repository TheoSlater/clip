/**
 * A numeric shade key used by HeroUI semantic palettes.
 */
type Shade =
    | "50"
    | "100"
    | "200"
    | "300"
    | "400"
    | "500"
    | "600"
    | "700"
    | "800"
    | "900";

/**
 * A HeroUI semantic colour palette (50-900).
 */
type SemanticPalette = Record<Shade, string>;

/**
 * The semantic colours object HeroUI expects inside a theme entry.
 * Kept intentionally tight to avoid accidental `any`.
 */
type HeroUiThemeColors = Partial<{
    background: string;
    foreground: string;
    divider: string;
    focus: string;
    content1: string;
    content2: string;
    content3: string;
    content4: string;
    default: SemanticPalette;
    primary: SemanticPalette;
    secondary: SemanticPalette;
}>;

/**
 * Your brand palettes derived from:
 * White Smoke:     #F7F7F7
 * Eigengrau:       #0C1421
 * Medium Sapphire: #3863A8
 * Aero:            #7CB9F2
 */
const defaultPalette: SemanticPalette = {
    "50": "#09090b",
    "100": "#18181b",
    "200": "#27272a",
    "300": "#3f3f46",
    "400": "#52525b",
    "500": "#71717a",
    "600": "#a1a1aa",
    "700": "#d4d4d8",
    "800": "#e4e4e7",
    "900": "#f4f4f5",
};

const primaryPalette: SemanticPalette = {
    "50": "#EAF0F9",
    "100": "#D7E0F2",
    "200": "#B6C6E6",
    "300": "#9BAED9",
    "400": "#6B87C2",
    "500": "#3863A8",
    "600": "#2F5490",
    "700": "#274577",
    "800": "#1F365E",
    "900": "#172845",
};

const secondaryPalette: SemanticPalette = {
    "50": "#F1F8FE",
    "100": "#E5F1FD",
    "200": "#CBE4FB",
    "300": "#B2D7F9",
    "400": "#8FC5F6",
    "500": "#7CB9F2",
    "600": "#6AA0CE",
    "700": "#5887AB",
    "800": "#476E87",
    "900": "#355564",
};

export const lightColors: HeroUiThemeColors = {
    // Layout tokens
    background: "#F7F7F7", // White Smoke
    foreground: "#0C1421", // Eigengrau
    divider: defaultPalette["200"],
    focus: secondaryPalette["500"],

    // Content surfaces
    content1: "#FFFFFF",
    content2: defaultPalette["50"],
    content3: defaultPalette["100"],
    content4: defaultPalette["200"],

    // Base semantic palettes
    default: defaultPalette,
    primary: primaryPalette,
    secondary: secondaryPalette,
};

export const darkColors: HeroUiThemeColors = {
    // Layout tokens
    background: "#0e0d10", // Eigengrau
    foreground: "#F7F7F7", // White Smoke
};
