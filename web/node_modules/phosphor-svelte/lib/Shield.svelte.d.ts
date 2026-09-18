import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShieldIcon` instead.
 *
 * @example
 * ```svelte
 * <Shield color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Shield: Component<IconComponentProps, {}, "">;
type Shield = ReturnType<typeof Shield>;
export default Shield;
