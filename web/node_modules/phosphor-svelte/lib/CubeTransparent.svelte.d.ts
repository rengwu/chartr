import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CubeTransparentIcon` instead.
 *
 * @example
 * ```svelte
 * <CubeTransparent color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CubeTransparent: Component<IconComponentProps, {}, "">;
type CubeTransparent = ReturnType<typeof CubeTransparent>;
export default CubeTransparent;
