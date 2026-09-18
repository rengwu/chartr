import { styleToString } from "./style.js";
export const srOnlyStyles = {
    position: "absolute",
    width: "1px",
    height: "1px",
    padding: "0",
    margin: "-1px",
    overflow: "hidden",
    clip: "rect(0, 0, 0, 0)",
    whiteSpace: "nowrap",
    borderWidth: "0",
    transform: "translateX(-100%)"
};
export const srOnlyStylesString = styleToString(srOnlyStyles);
