import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LockLaminatedIcon` instead.
 *
 * @example
 * ```svelte
 * <LockLaminated color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LockLaminated: Component<IconComponentProps, {}, "">;
type LockLaminated = ReturnType<typeof LockLaminated>;
export default LockLaminated;
