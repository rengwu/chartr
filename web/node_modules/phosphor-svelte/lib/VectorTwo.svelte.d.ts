import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VectorTwoIcon` instead.
 *
 * @example
 * ```svelte
 * <VectorTwo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const VectorTwo: Component<IconComponentProps, {}, "">;
type VectorTwo = ReturnType<typeof VectorTwo>;
export default VectorTwo;
