import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GhostIcon` instead.
 *
 * @example
 * ```svelte
 * <Ghost color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Ghost: Component<IconComponentProps, {}, "">;
type Ghost = ReturnType<typeof Ghost>;
export default Ghost;
