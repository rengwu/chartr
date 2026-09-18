import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SecurityCameraIcon` instead.
 *
 * @example
 * ```svelte
 * <SecurityCamera color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SecurityCamera: Component<IconComponentProps, {}, "">;
type SecurityCamera = ReturnType<typeof SecurityCamera>;
export default SecurityCamera;
