import {
	SetColorAtPixelAction,
	SetModificationInProgressAction,
	SetModifiedImageAction,
	SetOriginalImageAction,
} from '../store/types';

type Handler<S> = (state: S, action: ImageAction) => S;

export type ImageAction =
	| SetOriginalImageAction
	| SetModifiedImageAction
	| SetModificationInProgressAction
	| SetColorAtPixelAction;

export function createReducer<State>(
	initialState: State,
	handlers: { [type: string]: Handler<State> }
): Handler<State> {
	return (state: State = initialState, action: ImageAction): State => {
		if (Object.hasOwn(handlers, action.type)) {
			return handlers[action.type](state, action);
		}

		return state;
	};
}
