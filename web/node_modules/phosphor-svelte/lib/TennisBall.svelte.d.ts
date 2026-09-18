import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TennisBallIcon` instead.
 *
 * @example
 * ```svelte
 * <TennisBall color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TennisBall: Component<IconComponentProps, {}, "">;
type TennisBall = ReturnType<typeof TennisBall>;
export default TennisBall;
