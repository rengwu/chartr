import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WindIcon` instead.
 *
 * @example
 * ```svelte
 * <Wind color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Wind: Component<IconComponentProps, {}, "">;
type Wind = ReturnType<typeof Wind>;
export default Wind;
