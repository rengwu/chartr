import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RadicalIcon` instead.
 *
 * @example
 * ```svelte
 * <Radical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Radical: Component<IconComponentProps, {}, "">;
type Radical = ReturnType<typeof Radical>;
export default Radical;
