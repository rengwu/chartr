import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CirclesThreePlusIcon` instead.
 *
 * @example
 * ```svelte
 * <CirclesThreePlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CirclesThreePlus: Component<IconComponentProps, {}, "">;
type CirclesThreePlus = ReturnType<typeof CirclesThreePlus>;
export default CirclesThreePlus;
