import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SecurityCameraIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SecurityCameraIcon: Component<IconComponentProps, {}, "">;
type SecurityCameraIcon = ReturnType<typeof SecurityCameraIcon>;
export default SecurityCameraIcon;
