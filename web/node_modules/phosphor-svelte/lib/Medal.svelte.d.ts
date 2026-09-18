import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MedalIcon` instead.
 *
 * @example
 * ```svelte
 * <Medal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Medal: Component<IconComponentProps, {}, "">;
type Medal = ReturnType<typeof Medal>;
export default Medal;
