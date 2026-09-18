import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SirenIcon` instead.
 *
 * @example
 * ```svelte
 * <Siren color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Siren: Component<IconComponentProps, {}, "">;
type Siren = ReturnType<typeof Siren>;
export default Siren;
