import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CameraRotateIcon` instead.
 *
 * @example
 * ```svelte
 * <CameraRotate color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CameraRotate: Component<IconComponentProps, {}, "">;
type CameraRotate = ReturnType<typeof CameraRotate>;
export default CameraRotate;
