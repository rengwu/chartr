import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RugIcon` instead.
 *
 * @example
 * ```svelte
 * <Rug color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Rug: Component<IconComponentProps, {}, "">;
type Rug = ReturnType<typeof Rug>;
export default Rug;
