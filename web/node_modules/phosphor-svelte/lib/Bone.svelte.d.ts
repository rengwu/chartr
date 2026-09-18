import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BoneIcon` instead.
 *
 * @example
 * ```svelte
 * <Bone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bone: Component<IconComponentProps, {}, "">;
type Bone = ReturnType<typeof Bone>;
export default Bone;
