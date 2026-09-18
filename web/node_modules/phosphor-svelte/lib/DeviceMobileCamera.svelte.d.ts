import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DeviceMobileCameraIcon` instead.
 *
 * @example
 * ```svelte
 * <DeviceMobileCamera color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DeviceMobileCamera: Component<IconComponentProps, {}, "">;
type DeviceMobileCamera = ReturnType<typeof DeviceMobileCamera>;
export default DeviceMobileCamera;
