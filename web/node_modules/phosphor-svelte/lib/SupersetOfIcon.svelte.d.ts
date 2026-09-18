import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SupersetOfIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SupersetOfIcon: Component<IconComponentProps, {}, "">;
type SupersetOfIcon = ReturnType<typeof SupersetOfIcon>;
export default SupersetOfIcon;
