import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TrayIcon` instead.
 *
 * @example
 * ```svelte
 * <Tray color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tray: Component<IconComponentProps, {}, "">;
type Tray = ReturnType<typeof Tray>;
export default Tray;
