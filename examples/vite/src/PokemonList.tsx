import { useQuery } from "@apollo/client";
import { useState } from "react";
import Query from "./pokemonList.graphql";

const pageSize = 50;

export const PokemonList: React.FC = () => {
  const [page, setPage] = useState(0);
  const { data, error } = useQuery(Query, {
    variables: {
      limit: pageSize,
      offset: page * pageSize,
    },
  });

  if (error) {
    return <div>Error: {error.message}</div>;
  }

  if (!data) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <div>
        <button
          disabled={page === 0}
          onClick={() => setPage((page) => page - 1)}
        >
          Previous
        </button>
        <button
          disabled={data.species.length < pageSize}
          onClick={() => setPage((page) => page + 1)}
        >
          Next
        </button>
      </div>
      <ul className="pokemonList">
        {data.species.map((pokemon) => {
          const ja = pokemon.names.find((name) => name.language_id === 1);
          const en = pokemon.names.find((name) => name.language_id === 9);
          return (
            <li key={pokemon.id}>
              #{pokemon.id} <b>{ja?.name}</b> {en?.name}
            </li>
          );
        })}
      </ul>
    </div>
  );
};
