import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AxeIcon` instead.
 *
 * @example
 * ```svelte
 * <Axe color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Axe: Component<IconComponentProps, {}, "">;
type Axe = ReturnType<typeof Axe>;
export default Axe;
