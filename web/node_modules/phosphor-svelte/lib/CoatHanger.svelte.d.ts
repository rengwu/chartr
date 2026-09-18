import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CoatHangerIcon` instead.
 *
 * @example
 * ```svelte
 * <CoatHanger color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CoatHanger: Component<IconComponentProps, {}, "">;
type CoatHanger = ReturnType<typeof CoatHanger>;
export default CoatHanger;
