import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandFistIcon` instead.
 *
 * @example
 * ```svelte
 * <HandFist color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandFist: Component<IconComponentProps, {}, "">;
type HandFist = ReturnType<typeof HandFist>;
export default HandFist;
