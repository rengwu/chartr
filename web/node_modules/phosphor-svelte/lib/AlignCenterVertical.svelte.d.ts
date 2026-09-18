import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignCenterVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignCenterVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignCenterVertical: Component<IconComponentProps, {}, "">;
type AlignCenterVertical = ReturnType<typeof AlignCenterVertical>;
export default AlignCenterVertical;
