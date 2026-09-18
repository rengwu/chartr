import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GlobeXIcon` instead.
 *
 * @example
 * ```svelte
 * <GlobeX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GlobeX: Component<IconComponentProps, {}, "">;
type GlobeX = ReturnType<typeof GlobeX>;
export default GlobeX;
