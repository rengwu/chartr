import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BrowsersIcon` instead.
 *
 * @example
 * ```svelte
 * <Browsers color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Browsers: Component<IconComponentProps, {}, "">;
type Browsers = ReturnType<typeof Browsers>;
export default Browsers;
