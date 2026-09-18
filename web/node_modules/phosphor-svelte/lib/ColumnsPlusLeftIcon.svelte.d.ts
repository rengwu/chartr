import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ColumnsPlusLeftIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ColumnsPlusLeftIcon: Component<IconComponentProps, {}, "">;
type ColumnsPlusLeftIcon = ReturnType<typeof ColumnsPlusLeftIcon>;
export default ColumnsPlusLeftIcon;
