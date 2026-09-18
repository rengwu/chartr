import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotsSixVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <DotsSixVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsSixVertical: Component<IconComponentProps, {}, "">;
type DotsSixVertical = ReturnType<typeof DotsSixVertical>;
export default DotsSixVertical;
