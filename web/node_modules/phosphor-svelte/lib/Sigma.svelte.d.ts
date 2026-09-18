import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SigmaIcon` instead.
 *
 * @example
 * ```svelte
 * <Sigma color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sigma: Component<IconComponentProps, {}, "">;
type Sigma = ReturnType<typeof Sigma>;
export default Sigma;
