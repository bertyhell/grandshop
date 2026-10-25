import type { Reducer, UnknownAction } from '@reduxjs/toolkit';

type Handler<S> = (state: S, action: UnknownAction) => S;

export function createReducer<State>(
	initialState: State,
	handlers: { [type: string]: Handler<State> }
): Reducer<State> {
	return (state: State = initialState, action: UnknownAction): State => {
		if (Object.hasOwn(handlers, action.type)) {
			return handlers[action.type](state, action);
		}

		return state;
	};
}
