import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DogIcon` instead.
 *
 * @example
 * ```svelte
 * <Dog color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Dog: Component<IconComponentProps, {}, "">;
type Dog = ReturnType<typeof Dog>;
export default Dog;
