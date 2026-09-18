import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <DotsSixVerticalIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsSixVerticalIcon: Component<IconComponentProps, {}, "">;
type DotsSixVerticalIcon = ReturnType<typeof DotsSixVerticalIcon>;
export default DotsSixVerticalIcon;
