import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DetectiveIcon` instead.
 *
 * @example
 * ```svelte
 * <Detective color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Detective: Component<IconComponentProps, {}, "">;
type Detective = ReturnType<typeof Detective>;
export default Detective;
