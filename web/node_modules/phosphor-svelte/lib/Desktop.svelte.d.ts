import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DesktopIcon` instead.
 *
 * @example
 * ```svelte
 * <Desktop color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Desktop: Component<IconComponentProps, {}, "">;
type Desktop = ReturnType<typeof Desktop>;
export default Desktop;
