import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BrowserIcon` instead.
 *
 * @example
 * ```svelte
 * <Browser color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Browser: Component<IconComponentProps, {}, "">;
type Browser = ReturnType<typeof Browser>;
export default Browser;
