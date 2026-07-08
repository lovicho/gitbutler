import type { BundledTheme } from "shiki";

type PierreTheme =
	| "pierre-dark"
	| "pierre-dark-soft"
	| "pierre-dark-vibrant"
	| "pierre-dark-protanopia-deuteranopia"
	| "pierre-dark-tritanopia"
	| "pierre-light"
	| "pierre-light-soft"
	| "pierre-light-vibrant"
	| "pierre-light-protanopia-deuteranopia"
	| "pierre-light-tritanopia";

const knownDisplayNames: Record<string, string | undefined> = {
	andromeeda: "Andromeeda",
	"aurora-x": "Aurora X",
	"ayu-dark": "Ayu Dark",
	"ayu-light": "Ayu Light",
	"ayu-mirage": "Ayu Mirage",
	"catppuccin-frappe": "Catppuccin Frappé",
	"catppuccin-latte": "Catppuccin Latte",
	"catppuccin-macchiato": "Catppuccin Macchiato",
	"catppuccin-mocha": "Catppuccin Mocha",
	"dark-plus": "Dark+ (VS Code)",
	dracula: "Dracula",
	"dracula-soft": "Dracula Soft",
	"everforest-dark": "Everforest Dark",
	"everforest-light": "Everforest Light",
	"github-dark": "GitHub Dark",
	"github-dark-default": "GitHub Dark Default",
	"github-dark-dimmed": "GitHub Dark Dimmed",
	"github-dark-high-contrast": "GitHub Dark High Contrast",
	"github-light": "GitHub Light",
	"github-light-default": "GitHub Light Default",
	"github-light-high-contrast": "GitHub Light High Contrast",
	"gruvbox-dark-hard": "Gruvbox Dark Hard",
	"gruvbox-dark-medium": "Gruvbox Dark Medium",
	"gruvbox-dark-soft": "Gruvbox Dark Soft",
	"gruvbox-light-hard": "Gruvbox Light Hard",
	"gruvbox-light-medium": "Gruvbox Light Medium",
	"gruvbox-light-soft": "Gruvbox Light Soft",
	horizon: "Horizon",
	"horizon-bright": "Horizon Bright",
	houston: "Houston",
	"kanagawa-dragon": "Kanagawa Dragon",
	"kanagawa-lotus": "Kanagawa Lotus",
	"kanagawa-wave": "Kanagawa Wave",
	laserwave: "Laserwave",
	"light-plus": "Light+ (VS Code)",
	"material-theme": "Material",
	"material-theme-darker": "Material Darker",
	"material-theme-lighter": "Material Lighter",
	"material-theme-ocean": "Material Ocean",
	"material-theme-palenight": "Material Palenight",
	"min-dark": "Min Dark",
	"min-light": "Min Light",
	monokai: "Monokai",
	"night-owl": "Night Owl",
	"night-owl-light": "Night Owl Light",
	nord: "Nord",
	"one-dark-pro": "One Dark Pro",
	"one-light": "One Light",
	plastic: "Plastic",
	poimandres: "Poimandres",
	"pierre-dark": "Pierre Dark",
	"pierre-dark-soft": "Pierre Dark Soft",
	"pierre-dark-vibrant": "Pierre Dark Vibrant",
	"pierre-dark-protanopia-deuteranopia": "Pierre Dark Protanopia & Deuteranopia",
	"pierre-dark-tritanopia": "Pierre Dark Tritanopia",
	"pierre-light": "Pierre Light",
	"pierre-light-soft": "Pierre Light Soft",
	"pierre-light-vibrant": "Pierre Light Vibrant",
	"pierre-light-protanopia-deuteranopia": "Pierre Light Protanopia & Deuteranopia",
	"pierre-light-tritanopia": "Pierre Light Tritanopia",
	red: "Red",
	"rose-pine": "Rosé Pine",
	"rose-pine-dawn": "Rosé Pine Dawn",
	"rose-pine-moon": "Rosé Pine Moon",
	"slack-dark": "Slack Dark",
	"slack-ochin": "Slack Ochin",
	"snazzy-light": "Snazzy Light",
	"solarized-dark": "Solarized Dark",
	"solarized-light": "Solarized Light",
	"synthwave-84": "Synthwave '84",
	"tokyo-night": "Tokyo Night",
	vesper: "Vesper",
	"vitesse-black": "Vitesse Black",
	"vitesse-dark": "Vitesse Dark",
	"vitesse-light": "Vitesse Light",
} satisfies Record<BundledTheme | PierreTheme, string>;

export const displayName = (themeName: string): string | undefined => {
	const dn = knownDisplayNames[themeName];
	// oxlint-disable-next-line no-console
	if (dn === undefined) console.warn(`No display name for theme "${themeName}"`);
	return dn;
};
