import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CameraPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <CameraPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CameraPlus: Component<IconComponentProps, {}, "">;
type CameraPlus = ReturnType<typeof CameraPlus>;
export default CameraPlus;
