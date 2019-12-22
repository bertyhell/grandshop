import React, { FunctionComponent, ReactElement, ReactNode, useEffect, useReducer } from 'react';
import getUuid from 'uuid/v1';
import compact from 'lodash-es/compact';
import { connect } from 'react-redux';

import { applyModifications, ImageModificationInfo, saveImage, selectOriginalImage } from '../../helpers/interop-electron';
import Button from '../../components/Button/Button';
import { AppState } from '../../store/types';
import { selectModifiedImageUrl, selectOriginalImageUrl } from '../../store/selectors';
import { Select, SelectOption } from '../../components/Select/Select';
import AddBorder from '../../components/ImageModifications/AddBorder/AddBorder';
import { format } from '../../helpers/format';

import './ControlsSidebar.scss';

interface ControlsSidebarProps {
  originalImageUrl: string | null;
  modifiedImageUrl: string | null;
}

interface ActiveModifier {
  name: string;
  title: string;
  node: ReactNode;
  info: ImageModificationInfo | null;
  isOpen: boolean;
}

const ControlsSidebar: FunctionComponent<ControlsSidebarProps> = ({
                                                                    originalImageUrl,
                                                                    modifiedImageUrl
                                                                  }) => {

  const IMAGE_MODIFIER_COMPONENTS: { [modifierName: string]: ReactElement } = {
    addBorder: <AddBorder
        title=""
        onApply={() => {
        }}
        onToggle={() => {
        }}
        isOpen={true}/>
  };

  const IMAGE_MODIFIER_OPTIONS: SelectOption[] = [
    {
      label: "Voeg een gekleurde rand toe",
      value: "addBorder"
    }
  ];

  const IMAGE_MODIFIER_TITLES: { [modifierName: string]: string } = {
    addBorder: "Rand {0}"
  };

  type ActiveModifierAction =
      | { type: 'ADD', name: string }
      | { type: 'UPDATE', info: ImageModificationInfo, index: number }
      | { type: 'TOGGLE', index: number }

  const getModifierTitle = (activeModifiers: ActiveModifier[], modifierName: string): string => {
    return format(IMAGE_MODIFIER_TITLES[modifierName], activeModifiers.filter(mod => mod.name === modifierName).length + 1);
  };

  function activeModifiersReducer(activeModifiers: ActiveModifier[], action: ActiveModifierAction): ActiveModifier[] {
    let modifiers: ActiveModifier[] | undefined;
    switch (action.type) {
      case 'ADD':
        const title: string = getModifierTitle(activeModifiers, action.name);
        const newModifier = {
          title,
          node: React.cloneElement(
              IMAGE_MODIFIER_COMPONENTS[action.name],
              {
                title,
                onApply: (modifierInfo: ImageModificationInfo) => handleApplyModifier(modifierInfo, activeModifiers.length),
                onToggle: () => toggleModifier(activeModifiers.length),
                isOpen: true
              }
          ),
          info: null,
          name: action.name,
          isOpen: false,
          id: getUuid(),
        };
        return [
          ...activeModifiers.map(mod => {
            mod.isOpen = false; // Close all previous modifiers when adding a new one
            return mod;
          }),
          newModifier
        ];

      case 'UPDATE':
        modifiers = [...activeModifiers];
        modifiers[action.index].info = action.info;
        return modifiers;

      case 'TOGGLE':
        modifiers = [...activeModifiers];
        modifiers[action.index].isOpen = !modifiers[action.index].isOpen;
        return modifiers;
    }
  }

  const [activeModifiers, modifyActiveModifiers] = useReducer<React.Reducer<ActiveModifier[], ActiveModifierAction>>(activeModifiersReducer, []);

  useEffect(() => {
    const modifications: ImageModificationInfo[] = compact(activeModifiers.map((m: ActiveModifier) => m.info));
    if (modifications && modifications.length) {
      applyModifications(modifications); // TODO add spinner while modifications are in progress
    }
  }, [activeModifiers]);

  const handleApplyModifier = (modifierInfo: ImageModificationInfo, index: number) => {
    modifyActiveModifiers({ type: 'UPDATE', info: modifierInfo, index });
  };

  const toggleModifier = (index: number) => {
    modifyActiveModifiers({ type: 'TOGGLE', index });
  };

  const handleModifierSelectionChanged = (name: string) => {
    modifyActiveModifiers({ type: 'ADD', name });
  };

  return (
      <div className="m-controls-sidebar">
        <div className="m-button-wrapper">
          <Button onClick={() => {
            selectOriginalImage({
              buttonLabel: "Openen",
              // defaultPath // TODO default to the users images folder
              filters: [
                {
                  name: 'Alle bestanden',
                  extensions: ['*']
                },
                {
                  name: 'Afbeeldingen',
                  extensions: ['jpg', 'png', 'gif', 'tif', 'bmp', 'jpeg', 'webp']
                }
              ],
              title: 'Selecteer de afbeelding die je wil bewerken',
              properties: [
                // 'multiSelections', // TODO add multi image support
                'openFile'
              ]
            });
          }} label="Afbeelding openen"/>
        </div>
        {originalImageUrl &&
        <>
          <div className="m-button-wrapper">
            <Select
              options={IMAGE_MODIFIER_OPTIONS}
              placeholder="Select een bewerking"
              onChange={handleModifierSelectionChanged}
            />
          </div>
          <div className="m-modification-controls">
            {activeModifiers.map((m: ActiveModifier) => {
              return m.node;
            })}
          </div>
        </>
        }
        {modifiedImageUrl &&
        <div className="m-button-wrapper">
          <Button className="c-save-button" onClick={() => {
            saveImage({
              buttonLabel: "Opslaan",
              // defaultPath // TODO default to the users images folder
              filters: [
                {
                  name: 'Alle bestanden',
                  extensions: ['*']
                },
                {
                  name: 'Afbeeldingen',
                  extensions: ['jpg', 'png', 'gif', 'tif', 'bmp', 'jpeg', 'webp']
                }
              ],
              title: 'Kies de locatie en naam om je afbeelding op te slaan'
            });
          }} label="Afbeelding opslaan als"/>
        </div>
        }
      </div>
  );
};

const mapStateToProps = (state: { app: AppState }) => ({
  originalImageUrl: selectOriginalImageUrl(state),
  modifiedImageUrl: selectModifiedImageUrl(state)
});

export default connect(mapStateToProps)(ControlsSidebar);
