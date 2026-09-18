import { styleToCSS } from "./style-to-css.js";
export function styleToString(style = {}) {
    return styleToCSS(style).replace("\n", " ");
}
