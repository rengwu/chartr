import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BugBeetleIcon` instead.
 *
 * @example
 * ```svelte
 * <BugBeetle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BugBeetle: Component<IconComponentProps, {}, "">;
type BugBeetle = ReturnType<typeof BugBeetle>;
export default BugBeetle;
