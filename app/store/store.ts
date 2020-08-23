import { applyMiddleware, combineReducers, createStore, Reducer } from 'redux';
import thunk from 'redux-thunk';

import appReducer from './reducer';
import { AppState } from './types';

const middleware = [thunk];

export default createStore(
    combineReducers<Reducer<AppState>>({
      app: appReducer
    }),
    applyMiddleware(...middleware)
);
