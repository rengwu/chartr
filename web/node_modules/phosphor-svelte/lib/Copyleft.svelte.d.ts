import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CopyleftIcon` instead.
 *
 * @example
 * ```svelte
 * <Copyleft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Copyleft: Component<IconComponentProps, {}, "">;
type Copyleft = ReturnType<typeof Copyleft>;
export default Copyleft;
