import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MoonIcon` instead.
 *
 * @example
 * ```svelte
 * <Moon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Moon: Component<IconComponentProps, {}, "">;
type Moon = ReturnType<typeof Moon>;
export default Moon;
