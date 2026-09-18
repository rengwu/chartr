import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SunIcon` instead.
 *
 * @example
 * ```svelte
 * <Sun color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sun: Component<IconComponentProps, {}, "">;
type Sun = ReturnType<typeof Sun>;
export default Sun;
